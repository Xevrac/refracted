using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_RequestSurrender
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.RequestSurrender); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.RequestSurrender)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.RequestSurrender)) as Rts.CnC.Messages.Client.RequestSurrender;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);

            return value;
        }
        
    }
}
