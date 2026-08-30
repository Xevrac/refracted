using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_ProductAlert
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.ProductAlert); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.ProductAlert)obj;
            //  Serialize ProductId
            s.Write(value.ProductId);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.ProductAlert)) as Rts.CnC.Messages.Client.ProductAlert;
            //  Deserialize ProductId
            s.Read(out value.ProductId);

            return value;
        }
        
    }
}
