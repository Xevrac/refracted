using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_SimuCloud_GameReady_Element
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.SimuCloud.GameReady.Element); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.SimuCloud.GameReady.Element)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize array Token
            Rts.Serialization.Reference.Write(s, value.Token, () =>
            {
                s.WriteVarInt32(value.Token.Length);
                s.Write(value.Token, 0, value.Token.Length);
            });

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            Rts.CnC.Messages.SimuCloud.GameReady.Element value = default(Rts.CnC.Messages.SimuCloud.GameReady.Element);
            DeserializeValue(s, ref value);
            return value;
        }
        
        public static void DeserializeValue(System.IO.Stream s, ref Rts.CnC.Messages.SimuCloud.GameReady.Element value)
        {
            var valueRef = __makeref(value);
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize array Token
            Rts.Serialization.Reference.Read(s, out value.Token, () =>
            {
                int length = s.ReadVarInt32();
                System.Byte[] tmp = new System.Byte[length];
                s.Read(tmp, 0, length);
                return tmp;
            });

        }
    }
}
