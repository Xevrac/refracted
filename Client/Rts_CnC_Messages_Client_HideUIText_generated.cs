using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_HideUIText
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.HideUIText); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.HideUIText)obj;
            //  Serialize array TextIds
            Rts.Serialization.Reference.Write(s, value.TextIds, () =>
            {
                s.WriteVarInt32(value.TextIds.Length);
                for(int i = 0 ; i < value.TextIds.Length ; ++i)
                {
                    s.Write(value.TextIds[i]);
                }
            });

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.HideUIText)) as Rts.CnC.Messages.Client.HideUIText;
            //  Deserialize array TextIds
            Rts.Serialization.Reference.Read(s, out value.TextIds, () =>
            {
                int length = s.ReadVarInt32();
                System.UInt32[] tmp = new System.UInt32[length];
                for(int i = 0 ; i < length ; ++i)
                {
                    s.Read(out tmp[i]);
                }
                return tmp;
            });

            return value;
        }
        
    }
}
