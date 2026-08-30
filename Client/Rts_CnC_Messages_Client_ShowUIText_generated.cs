using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_ShowUIText
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.ShowUIText); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.ShowUIText)obj;
            //  Serialize array TextIds
            Rts.Serialization.Reference.Write(s, value.TextIds, () =>
            {
                s.WriteVarInt32(value.TextIds.Length);
                for(int i = 0 ; i < value.TextIds.Length ; ++i)
                {
                    s.Write(value.TextIds[i]);
                }
            });
            //  Serialize DurationMs
            s.Write(value.DurationMs);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.ShowUIText)) as Rts.CnC.Messages.Client.ShowUIText;
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
            //  Deserialize DurationMs
            s.Read(out value.DurationMs);

            return value;
        }
        
    }
}
