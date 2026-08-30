using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_GameVictory
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.GameVictory); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.GameVictory)obj;
            //  Serialize TextId
            s.Write(value.TextId);
            //  Serialize array Parameters
            Rts.Serialization.Reference.Write(s, value.Parameters, () =>
            {
                s.WriteVarInt32(value.Parameters.Length);
                for(int i = 0 ; i < value.Parameters.Length ; ++i)
                {
                    s.Write(value.Parameters[i]);
                }
            });
            //  Serialize HasParameters
            s.Write(value.HasParameters);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.GameVictory)) as Rts.CnC.Messages.Client.GameVictory;
            //  Deserialize TextId
            s.Read(out value.TextId);
            //  Deserialize array Parameters
            Rts.Serialization.Reference.Read(s, out value.Parameters, () =>
            {
                int length = s.ReadVarInt32();
                System.Int32[] tmp = new System.Int32[length];
                for(int i = 0 ; i < length ; ++i)
                {
                    s.Read(out tmp[i]);
                }
                return tmp;
            });
            //  Deserialize HasParameters
            s.Read(out value.HasParameters);

            return value;
        }
        
    }
}
